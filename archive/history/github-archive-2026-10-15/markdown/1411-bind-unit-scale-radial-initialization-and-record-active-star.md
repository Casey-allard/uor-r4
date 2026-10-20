# #1411 — Bind unit-scale radial initialization and record active startup paths

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T02:38:21Z
- **Updated:** 2026-10-09T13:17:49Z
- **Closed:** 2026-09-27T02:39:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1411
- **Merged:** 2026-09-27T02:39:37Z
- **Merge commit:** 45cd29ff61577d0b3b2922bbf0c585f345b60606
- **Base / Head:** main / UOR-Foundation:codex/radial-unit-start-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fresh radial campaigns previously inherited the constructor's beta≈12.92 without an explicit alternative in campaign provenance. Add optional typed `read_initialization: "unit_scale"` so Lorentz and LorentzAffine can start at beta=1 through the actual `joint-fit` path. The absent default retains existing serialization and constructor behavior. Resume loads the evolved scalar and rejects changed/missing initializer lineage, including quantization/projection transitions. Pinned model numerical contracts remain unchanged; report metadata records the override separately.

Add a fixed, no-update natural-prefix startup example and its source-bound evidence. Six focused checks pass: four initializer/provenance/resume checks and two retained calibrated-checkpoint checks. Library checks used source `17f0cde0`; final example source is `0aa932dc`. Formatting, whitespace and claim-wording checks pass. Compile/check commands total488.98seconds; no broad suite or fitted-model study was run.

Antigravity executed the exact startup example from this worktree. The fourth lab independently verified its source/executable, every sealed file, actual evaluator/dev/tokenizer identities and all four full256 windows, preserving the report and executable instead of duplicating the run. Both arms have finite nonzero gradients in all six required parameter families and no zero causal-read positions. This202.23second untrained observation establishes an active initializer path only. External launcher thread limits and true peak RSS were not verified. Supplementary generation has a disclosed `[0,0]` BOS prefix and incoherent eight-token outputs; it supplies no language-quality evidence.

Update the canonical next action to explicit shared-parameter transfer from a retained Dot checkpoint into both radial readers, with new lineage and identical optimizer resets, before selecting a meaningful adaptation dose. That transfer is not implemented or launched here. The independently owned termination-objective study remains separate. Record the concurrent Google-branch integration/evidence findings without altering its source.

References #973, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

