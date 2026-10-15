# #1242 — Measure matched context augmentation with the geometric decoder fixed

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T04:40:08Z
- **Updated:** 2026-10-09T13:17:20Z
- **Closed:** 2026-09-13T04:45:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1242
- **Merged:** 2026-09-13T04:45:56Z
- **Merge commit:** b506820d14e03b522d4b6f41ec73437f61733c2b
- **Base / Head:** main / UOR-Foundation:codex/geometric-context-augmentation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The previous scan found improvements supported by very few construction occurrences. This adds a Rust-only matched context-augmentation experiment to test broader support with the same frozen decoder: two 8,192-proposal fits, equal full-causal exposure, original repetition versus length-matched variants, and source-bound state witnesses.

Actual result: `FAIL_CONTEXT_AUGMENTATION_DEVELOPMENT_GATE`. Variation widens update support but worsens the recombination probe versus parent and repetition. Both arms fail preservation/useful generation; retain 15baec48. Production source and geometry are unchanged. The result and current state recommend four controlled conditional decoder fits on the frozen states to separate decoder adaptation from state-learning benefit; those fits are NOT_RUN.

Validation: release build plus two focused tests, separate data preparation, actual paired experiment, exact replay of 15,795 prior rows and 60 outputs, independent static result review, formatting, claim wording, diff and 139-source/21-report-file verification. Seventeen older seals/668 files, parked changes and retained model verified. Model/build/preparation charge 135,526 ms; no paid compute or cleanup. GitHub status names are compatibility acknowledgements, not local test results.

Parent #1241 merged at e0eff36bfcbfd786a2005373bbac319c8271a1c2 with source/tree equality verified. This PR now targets main for normal protected delivery.

References #973, #964, #820.

The parent squash merge required ancestry reconciliation. Delivery commit 4afc126 preserves the exact tested 14e3b29a tree, 22ef354d4761a94ba6c909bb607fe4506903a609; no source/model change or repeated fit.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

