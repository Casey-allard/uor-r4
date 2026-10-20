# #1250 — Record failed matched causal-credit learning pilot

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T16:45:19Z
- **Updated:** 2026-10-09T13:17:05Z
- **Closed:** 2026-09-13T16:46:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1250
- **Merged:** 2026-09-13T16:46:56Z
- **Merge commit:** 126e139e09470cc716726904f46c223ff712651e
- **Base / Head:** main / UOR-Foundation:codex/addressed-attention-causal-pilot
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

One matched 64-update causal-credit pilot completes but fails generation and retention acceptance: development CE worsens from 5.5530 to 5.5711, with zero exact answers across 24 training/development records. Preserve the new checkpoint and compiled witness as negative evidence; retain 15baec48.

Adds a separately source-bound causal checkpoint codec and matched Rust fit/evaluation drivers, preserving old artifacts and comparing every output against both predecessor endpoints. Canonical state now calls for a deterministic-forward credit contract and finite whole-runtime causal test before further fitting.

Validation: 50 focused release tests; separate fit/evaluation executions; all-24-record interpreted/compiled trace parity; independent evidence review; source/sealed-report/preservation checks. Local checks carry validation; queue acknowledgements do not. Complete model/build charge 137542/240000ms; no external compute or promotion.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

