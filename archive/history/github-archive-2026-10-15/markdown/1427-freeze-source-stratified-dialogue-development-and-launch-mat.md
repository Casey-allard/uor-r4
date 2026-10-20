# #1427 — Freeze source-stratified dialogue development and launch matched learning study

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T08:56:36Z
- **Updated:** 2026-10-09T13:17:15Z
- **Closed:** 2026-09-27T08:58:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1427
- **Merged:** 2026-09-27T08:58:31Z
- **Merge commit:** 0ac3df4679dd0696c4d1f56a231ac96e805ee3af
- **Base / Head:** main / UOR-Foundation:codex/dialogue-prefix-study-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The complete-prefix dialogue learner previously sampled a small aggregate development panel, leaving source-specific behavior underrepresented. Add deterministic source-stratified selection, target-counted per-source losses and a preparation mode that seals exact development IDs before either arm can learn. Fits bind and regenerate that same panel; existing campaigns without the optional fields retain their development contract.

Freeze and launch one 1,024-update full-prefix versus role-only comparison from the retained R1d. Both arms share response/EOS exposure and the 161-response development inventory, then reload saved weights and generate the same 58 existing open replies. The evidence packet records the actual build, data-only preparation, independent design/launch reviews, full resource projection and a live 51-update snapshot. This PR delivers source and the running study contract; completed fits, useful chat and model promotion remain pending. References #973 and #820.

Validation executed locally: three focused selection/mask/aggregation Rust tests and the release `cpu-accelerate` example build passed (56.360 seconds combined); `cargo fmt --all --check`, `git diff --check` and claim-wording check passed. The native preparation sealed 161 unique responses with zero forward/update/generation work. Eighteen stable publication bindings match; two mutable process receipts are explicitly dated snapshots. Queue compatibility acknowledgements are not test results.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

