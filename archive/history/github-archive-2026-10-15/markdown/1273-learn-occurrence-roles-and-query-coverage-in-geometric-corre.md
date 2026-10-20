# #1273 — Learn occurrence roles and query coverage in geometric correspondence

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T13:23:38Z
- **Updated:** 2026-10-09T13:17:38Z
- **Closed:** 2026-09-14T13:24:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1273
- **Merged:** 2026-09-14T13:24:55Z
- **Merge commit:** f5c730b76863ffbfaa5d0ba18d4a908adc54173b
- **Base / Head:** main / UOR-Foundation:codex/learned-occurrence-roles
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

A word inherited as context-only could never be selected as a name payload. After learning occurrence roles, the old suffix-coverage predicate could still accept a source that matched only the final name while omitting the requested relation. For example, `who did help will?` could admit `alice` from `will will call alice.` even when `helen did help will.` was available.

This adds an experimental artifact-bound source/query role learner over canonical neighbor identities, with role agreement inside each injective correspondence witness. Actual output-compatible span alternatives remain latent. A separate discrete output fit learns available-query coverage, retaining the role tables and parent reader/updater/completion/writer. Serving remains bounded geometric/integer/table execution. The original path and default model are preserved.

The final authored gate is **PASS_LEARNED_OCCURRENCE_ROLE**: source/query training 504/504 and 312/312; mixed-role 72/72; earlier lexical 48/48 and phrase-order 80/80; all 6,688 earlier complete traces retained, including 512 typed unresolved cases and 384 most recent correspondence rows. ExactIdentity/reload agree on 200/200 complete structures. Removing query agreement drops mixed-role success to 48/72; removing coverage drops it to 36/72 and reproduces all 200 prior candidate structures. Read/update controls lose every correct dependent answer. Two failed candidates and all original evidence are retained.

Validation: four optimized offline uor-r4-core builds with 33 focused dependent-language tests each; actual fits/generation/controls; artifact roundtrip and malformed query-key rejection; formatting, capability wording, diff and preservation checks. The evidence record binds three sealed attempts and four source/binary versions. Queue CI acknowledgements are not test execution.

This qualifies bounded lexical-role recombination and available-anchor coverage on authored development inputs. General prose, unseen grammatical necessity, independent final holdout and model promotion remain unqualified. The next frozen-artifact test separates required query evidence from a question word that merely appears in an irrelevant record, alongside styled role recombination. Roadmap/current-state/map and local resources/restart notes are updated.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

