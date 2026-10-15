# #1648 — Preserve unqualified delta-rule prototype and negative research evidence

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T16:08:22Z
- **Updated:** 2026-10-09T13:17:13Z
- **Closed:** 2026-10-03T19:19:31Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1648
- **Merged:** 2026-10-03T19:19:31Z
- **Merge commit:** 4a7c42d9daa33aeb303ac1e6d01cd5975c53110b
- **Base / Head:** main / UOR-Foundation:codex/delta-research-delivery-20261003
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

DeepSeek’s delta-rule prototype, original design and six negative-result logs were held on codex/delta-rule-memory rather than main. Preserve them under prototypes/ with an explicit unqualified-research banner. No production crate, dependency, model selector or roadmap adopts the prototype.

The full-model embedding gradient remains defective/unverified. The best historical authored-task hit rate is 7.03% versus the exact store’s 100%; native lowering and capacity/accounting claims remain hypotheses. No new fit was run.

Validation: standalone Rust metadata compilation succeeds with six warnings; this is compile-only, not gradient/model validation. Claim-wording and whitespace checks pass. Independent exact-head review at ae74e2c9 verifies original Rust and all six logs byte-identical to source00efb690 and approves preservation-only delivery. References #1512, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

