# #1582 — Preserve geometric binding under hard capture and hold

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T19:50:45Z
- **Updated:** 2026-10-09T13:17:09Z
- **Closed:** 2026-10-01T20:03:27Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1582
- **Merged:** 2026-10-01T20:03:27Z
- **Merge commit:** ad820677696a7512af6a58d2d5d18f8fb923ad4c
- **Base / Head:** main / UOR-Foundation:codex/attention-hard-bridge-20261001
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The saved geometric-attention models previously demonstrated binding only with soft identity retention. Add explicit per-call hard HOLD/CAPTURE diagnostics through the same Lorentz read, without changing weights, ordinary soft behavior, candidate access or saved model formats.

Both successful Held seeds retain 128/128 answers and correct-source majorities on each original/longer-gap panel with zero answer changes; maximum Held answer-logit drift 9.60231e-5. Matched Local answers remain unchanged. All four saved models were evaluated with zero training, original inputs retained, outputs sealed, exact executable/source/model hashes preserved. This establishes gate-discretization fidelity on authored development panels; remaining computation is float/full-prefix. It is not integer serving or language qualification.

Validation: actual model evaluation 125.779s; nine focused latch tests passed (new hard semantics/causality/label isolation/immutability/reload and retained soft/format/export boundaries). Touched Rust rustfmt, diff check and claim-wording passed. The final evaluator configuration guard is compiled separately; the measured earlier executable and source remain retained. Independent mathematical/source/row review found no blocker. Report/current state/plan advance to scale-aware integer latch lowering.

References #1512.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

